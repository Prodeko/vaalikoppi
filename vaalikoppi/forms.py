from django import forms

from vaalikoppi.models import Voting, RankedChoiceVoting


class VotingForm(forms.ModelForm):
    class Meta:
        model = Voting
        fields = [
            "voting_name",
            "voting_description",
            "max_votes",
            "is_password_protected",
            "voting_password",
        ]
        labels = {
            "voting_name": "Äänestyksen nimi",
            "voting_description": "Äänestyksen kuvaus",
            "max_votes": "Ääniä käytössä",
            "is_password_protected": "Äänestyskohtainen salasana vaaditaan",
            "voting_password": "Äänestyskohtainen salasana",
        }


class RankedChoiceVotingForm(forms.ModelForm):
    class Meta:
        model = RankedChoiceVoting
        fields = [
            "voting_name",
            "voting_description",
            "max_votes",
            "is_password_protected",
            "voting_password",
        ]
        labels = {
            "voting_name": "Äänestyksen nimi",
            "voting_description": "Äänestyksen kuvaus",
            "max_votes": "Ääniä käytössä",
            "is_password_protected": "Äänestyskohtainen salasana vaaditaan",
            "voting_password": "Äänestyskohtainen salasana",
        }
